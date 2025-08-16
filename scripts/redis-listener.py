#!/usr/bin/env python3
"""
Redis listener script to test the Uniswap Relay DApp pub/sub functionality.
This script subscribes to the swap_events channel and displays incoming events.
"""

import redis
import json
import time
import sys
from datetime import datetime

def format_event(event_data):
    """Format event data for display"""
    try:
        event = json.loads(event_data)
        
        # Extract key information
        chain = event.get('chain', 'Unknown')
        dex = event.get('dex', 'Unknown')
        tx_hash = event.get('transaction_hash', 'Unknown')[:10] + '...'
        token_in = event.get('token_in', {}).get('symbol', 'Unknown')
        token_out = event.get('token_out', {}).get('symbol', 'Unknown')
        amount_in = event.get('amount_in', 'Unknown')
        amount_out = event.get('amount_out', 'Unknown')
        timestamp = event.get('timestamp', 'Unknown')
        
        return f"[{timestamp}] {chain} | {dex} | {tx_hash} | {amount_in} {token_in} → {amount_out} {token_out}"
    
    except json.JSONDecodeError:
        return f"Raw data: {event_data[:100]}..."

def main():
    """Main function to listen for Redis events"""
    print("🚀 Uniswap Relay DApp - Redis Event Listener")
    print("=" * 60)
    
    # Redis connection configuration
    redis_host = 'localhost'
    redis_port = 6379
    redis_channel = 'swap_events'
    
    try:
        # Connect to Redis
        print(f"📡 Connecting to Redis at {redis_host}:{redis_port}...")
        r = redis.Redis(host=redis_host, port=redis_port, decode_responses=True)
        
        # Test connection
        r.ping()
        print("✅ Redis connection established")
        
        # Subscribe to channel
        print(f"🎧 Subscribing to channel: {redis_channel}")
        print("⏳ Waiting for events... (Press Ctrl+C to exit)")
        print("-" * 60)
        
        pubsub = r.pubsub()
        pubsub.subscribe(redis_channel)
        
        event_count = 0
        start_time = time.time()
        
        for message in pubsub.listen():
            if message['type'] == 'message':
                event_count += 1
                elapsed = time.time() - start_time
                
                print(f"\n📊 Event #{event_count} (after {elapsed:.1f}s)")
                print(format_event(message['data']))
                
                # Optional: Save to file
                with open('events.log', 'a') as f:
                    f.write(f"{datetime.now().isoformat()} | {message['data']}\n")
                
            elif message['type'] == 'subscribe':
                print(f"✅ Subscribed to {message['channel']}")
                
    except redis.ConnectionError:
        print(f"❌ Failed to connect to Redis at {redis_host}:{redis_port}")
        print("Make sure Redis is running and accessible")
        sys.exit(1)
        
    except KeyboardInterrupt:
        print("\n\n👋 Shutting down...")
        if 'pubsub' in locals():
            pubsub.close()
        if 'r' in locals():
            r.close()
        print(f"📈 Total events received: {event_count}")
        sys.exit(0)
        
    except Exception as e:
        print(f"❌ Error: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main() 