package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/url"
)

type GeoResponse struct {
    // A list of results; we only need the first one
    Results []LatLong `json:"results"`
}

type LatLong struct {
    Latitude  float64 `json:"latitude"`
    Longitude float64 `json:"longitude"`
}

func getLatLong(city string) (*LatLong, error) {
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

//we'll now make a new function that will take a LatLong and retun the weather forecast
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

func main() {
	latlong, err := getLatLong("Montreal")
	if err != nil {
		log.Fatalf("Failed to get lat long: %s", err)
	}
	fmt.Printf("Latitude: %f, Longitude: %f", latlong.Latitude, latlong.Longitude)
	weather, err := getWeather(*latlong)
	if err != nil {
		log.Fatalf("Failed to get weather: %s", err)
	}
	fmt.Printf("Weather: %s\n", weather)
}