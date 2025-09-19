package main

import (
    "encoding/json"
    "fmt"
    "net/http"
    "log"
)

type ServiceMessage struct {
    ServiceID string          `json:"service_id"`
    Version   string          `json:"version"`
    Payload   json.RawMessage `json:"payload"`
    Timestamp int64           `json:"timestamp"`
}

func main() {
    http.HandleFunc("/health", healthHandler)
    http.HandleFunc("/message", messageHandler)
    
    fmt.Println("[go-service] P2P Service listening on :3002")
    log.Fatal(http.ListenAndServe(":3002", nil))
}

func healthHandler(w http.ResponseWriter, r *http.Request) {
    response := map[string]interface{}{
        "service_id": "go-service",
        "healthy":    true,
        "version":    "v1.stable.0.stable.0.stable",
    }
    json.NewEncoder(w).Encode(response)
}

func messageHandler(w http.ResponseWriter, r *http.Request) {
    var msg ServiceMessage
    json.NewDecoder(r.Body).Decode(&msg)
    fmt.Printf("Received message from %s\n", msg.ServiceID)
    w.Write([]byte("Message received"))
}
