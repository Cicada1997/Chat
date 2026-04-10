import socket
import threading

def receive_messages(sock):
    """Wait for messages from the server and print them."""
    while True:
        try:
            data = sock.recv(1024)
            if not data:
                print("\n[ Connection lost ]")
                break
            # Decode and strip the newline we sent earlier
            print(f"\rBroadcast: {data.decode('utf-8').strip()}\n> ", end="")
        except Exception as e:
            print(f"\n[ Error receiving: {e} ]")
            break

def start_client():
    host = '127.0.0.1'
    port = 5225

    client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    
    try:
        client_socket.connect((host, port))
        print(f"--- Connected to CicadaChat at {host}:{port} ---")
    except ConnectionRefusedError:
        print("Connection failed. Is the Rust server running?")
        return

    # Start a background thread to listen for broadcasts
    threading.Thread(target=receive_messages, args=(client_socket,), daemon=True).start()

    try:
        while True:
            msg = input("> ")
            if msg.lower() in ['exit', 'quit']:
                break
            
            # CRITICAL: Append '\n' because Rust's read_line() looks for it
            client_socket.sendall(f"{msg}\n".encode('utf-8'))
    except KeyboardInterrupt:
        pass
    finally:
        print("\nClosing connection...")
        client_socket.close()

if __name__ == "__main__":
    start_client()
