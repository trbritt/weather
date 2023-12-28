package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"os"
	"time"

	"github.com/gin-gonic/gin"
	_ "github.com/jackc/pgx/v5/stdlib" // Standard library bindings for pgx
	"github.com/jmoiron/sqlx"
)

type GeoResponse struct {
	// A list of results; we only need the first one
	Results []LatLong `json:"results"`
}

type LatLong struct {
	Latitude  float64 `json:"latitude"`
	Longitude float64 `json:"longitude"`
}

type WeatherResponse struct {
	Latitude  float64 `json:"latitude"`
	Longitude float64 `json:"longitude"`
	Timezone  string  `json:"timezone"`
	Hourly    struct {
		Time          []string  `json:"time"`
		Temperature2m []float64 `json:"temperature_2m"`
	} `json:"hourly"`
}

type WeatherDisplay struct {
	City      string
	Forecasts []Forecast
}

type Forecast struct {
	Date        string
	Temperature string
}

// this will insert values into our local postgres db for caching
func insertCity(db *sqlx.DB, name string, latLong LatLong) error {
	_, err := db.Exec("INSERT INTO cities (name, latitude, longitude) VALUES ($1, $2, $3)", name, latLong.Latitude, latLong.Longitude)
	return err
}

func fetchLatLong(city string) (*LatLong, error) {
	endpoint := fmt.Sprintf("https://geocoding-api.open-meteo.com/v1/search?name=%s&count=1&language=en&format=json", url.QueryEscape(city))
	resp, err := http.Get(endpoint)
	if err != nil {
		return nil, fmt.Errorf("error making request to Geo API: %w", err)
	}
	//The defer statement ensures that the response body is closed after the function returns.
	//This is a common pattern in Go to avoid resource leaks. The compiler does not warn us
	//in case we forget, so we need to be careful here
	defer resp.Body.Close()

	var response GeoResponse
	if err := json.NewDecoder(resp.Body).Decode(&response); err != nil {
		return nil, fmt.Errorf("error decoding repsonse: %w", err)
	}
	if len(response.Results) < 1 {
		return nil, errors.New("no results found")
	}
	return &response.Results[0], nil
}

func getLatLong(db *sqlx.DB, name string) (*LatLong, error) {
	var latLong *LatLong
	qryStr := fmt.Sprintf("SELECT latitude,longitude FROM cities WHERE name='%s'", name)
	// err := db.Get(&latLong, qryStr)
	rows, err := db.Query(qryStr)
	if err == nil {
		for rows.Next() {
			// fmt.Println(rows)
			// err = rows.StructScan(latLong)
			var local_lat float64
			var local_long float64
			err = rows.Scan(&local_lat, &local_long)
			if err == nil {
				local_latLong := LatLong{
					Latitude:  local_lat,
					Longitude: local_long,
				}
				return &local_latLong, nil
			}
		}
	}
	latLong, err = fetchLatLong(name)
	if err != nil {
		return nil, err
	}
	err = insertCity(db, name, *latLong)
	if err != nil {
		return nil, err
	}
	return latLong, nil
}

// we'll now make a new function that will take a LatLong and retun the weather forecast
func getWeather(latLong LatLong) (string, error) {
	endpoint := fmt.Sprintf("https://api.open-meteo.com/v1/forecast?latitude=%.6f&longitude=%.6f&hourly=temperature_2m", latLong.Latitude, latLong.Longitude)
	resp, err := http.Get(endpoint)
	if err != nil {
		return "", fmt.Errorf("error making request to weather api: %w", err)
	}
	defer resp.Body.Close()
	body, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("error reading response body: %w", err)
	}
	return string(body), nil
}

// lets now make a function that parses the weather JSON output into a more friendly format
func extractWeatherData(city string, rawWeather string) (WeatherDisplay, error) {
	var weatherResponse WeatherResponse
	if err := json.Unmarshal([]byte(rawWeather), &weatherResponse); err != nil {
		return WeatherDisplay{}, fmt.Errorf("error decoding weather response: %w", err)
	}
	var forecasts []Forecast
	for i, t := range weatherResponse.Hourly.Time {
		date, err := time.Parse("2006-01-02T15:04", t)
		if err != nil {
			return WeatherDisplay{}, err
		}
		forecast := Forecast{
			Date:        date.Format("Mon 15:04"),
			Temperature: fmt.Sprintf("%.1f°C", weatherResponse.Hourly.Temperature2m[i]),
		}
		forecasts = append(forecasts, forecast)
	}
	return WeatherDisplay{
		City:      city,
		Forecasts: forecasts,
	}, nil
}
func getLastCities(db *sqlx.DB) ([]string, error) {
    var cities []string
    err := db.Select(&cities, "SELECT name FROM cities ORDER BY id DESC LIMIT 10")
    if err != nil {
   	 return nil, err
    }
    return cities, nil
}
func main() {
	// db := sqlx.MustConnect("postgres", os.Getenv("DATABASE_URL"))
	db, err := sqlx.Connect("pgx", os.Getenv("DATABASE_URL"))
	if err != nil {
		fmt.Errorf("Could not connect to db: %w", err)
	}
	r := gin.Default()
	r.LoadHTMLGlob("views/*")
	r.GET("/stats", gin.BasicAuth(gin.Accounts{
		"forecast": "forecast",
		}), func(c *gin.Context) {
		cities, err := getLastCities(db)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		c.HTML(http.StatusOK, "stats.html", cities)
	})
	r.GET("/", func(c *gin.Context) {
		c.HTML(http.StatusOK, "index.html", nil)
	})
	r.GET("/weather", func(c *gin.Context) {
		city := c.Query("city")
		latlong, err := getLatLong(db, city)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		weather, err := getWeather(*latlong)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		// c.JSON(http.StatusOK, gin.H{"weather" : weather})
		weatherDisplay, err := extractWeatherData(city, weather)
		if err != nil {
			c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
			return
		}
		c.HTML(http.StatusOK, "weather.html", weatherDisplay)
	})
	r.Run()

	// below is a static test
	// latlong, err := getLatLong("Montreal")
	// if err != nil {
	// 	log.Fatalf("Failed to get lat long: %s", err)
	// }
	// fmt.Printf("Latitude: %f, Longitude: %f", latlong.Latitude, latlong.Longitude)
	// weather, err := getWeather(*latlong)
	// if err != nil {
	// 	log.Fatalf("Failed to get weather: %s", err)
	// }
	// fmt.Printf("Weather: %s\n", weather)
}
