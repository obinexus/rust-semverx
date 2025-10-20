from flask import Flask, request, jsonify
import requests
import json
from datetime import datetime

app = Flask(__name__)

class P2PService:
    def __init__(self, service_id, port):
        self.service_id = service_id
        self.port = port
        self.peers = {}
        self.version = "v1.stable.0.experimental.0.stable"
        
    def connect_peer(self, peer_id, peer_addr):
        self.peers[peer_id] = peer_addr
        
    def send_to_peer(self, peer_id, message):
        if peer_id in self.peers:
            url = f"http://{self.peers[peer_id]}/message"
            return requests.post(url, json=message)
        return None

service = P2PService("python-service", 3003)

@app.route('/health')
def health():
    return jsonify({
        'service_id': service.service_id,
        'healthy': True,
        'version': service.version
    })

@app.route('/message', methods=['POST'])
def message():
    msg = request.json
    print(f"Received message from {msg.get('service_id')}")
    return "Message received"

if __name__ == '__main__':
    # Connect to other services
    service.connect_peer("rust-service", "127.0.0.1:3001")
    service.connect_peer("go-service", "127.0.0.1:3002")
    
    print(f"[python-service] P2P Service listening on :{service.port}")
    app.run(port=service.port)
