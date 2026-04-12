import socket
import threading
import json
import traceback

from dotenv import load_dotenv
import os

from typing import Dict

running = True

def receive_messages(sock):
    global running
    """Wait for messages from the server and print them."""
    while True:
        try:
            data = sock.recv(1024)
            if not data:
                print("\n[ Connection lost ]")
                break

            try:
                json_packet: Dict = json.loads(data.decode('utf-8').strip())

            except Exception as e:
                print(e)
                continue


            match list(json_packet.keys())[0]:
                case "NewMessage":
                    msg = json_packet["NewMessage"]
                    name = msg["username"]
                    print(f"\r{ name if name else msg["author_id"] }: {msg["content"]}\n> ", end="")
                
                case "Disconnect":
                    disconn_msg = json_packet["Disconnect"]
                    print(f"\rYou have been disconnected from the server. reason: {disconn_msg["reason"]}")
                    print("\n\tPress enter to exit...")
                    running = False
                    break

                case _:
                    print(f"\rCould not parse packet:\n{ json.dumps(json_packet) }\n> ", end="")
                    continue

        except Exception as e:
            print(traceback.format_exc(e))
            break

def create_msg(content: str):
    return {
        "SendMessage": {
            "content": content,
            "channel_id": 0,
        }
    }

def login(client):
    load_dotenv()
    token = os.getenv("KATTAUTH_TOKEN")

    if token:
        client.sendall((json.dumps({
            "TokenLogin": {
                "token": token,
            }
        }) + "\n").encode("utf-8"))

    else:
        username = input("\rEnter kattauth username: ")
        password = input("\rEnter kattauth password: ")

        client.sendall((json.dumps({
            "Login": {
                "username": username,
                "password": password,
            }
        }) + "\n")
            .encode("utf-8"))

def start_client():
    host = '127.0.0.1'
    port = 19975

    client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    try:
        client_socket.connect((host, port))
        print(f"--- Connected to CicadaChat at {host}:{port} ---")

    except ConnectionRefusedError:
        print("Connection failed. Is the server running?")
        return

    threading.Thread(target=receive_messages, args=(client_socket,), daemon=True).start()

    login(client_socket)

    try:
        while running:
            msg = input("> ")
            if msg.lower() in ['exit', 'quit']:
                break
            
            client_socket.sendall(f"{json.dumps(create_msg(msg))}\n".encode("utf-8"))

    except KeyboardInterrupt:
        pass

    finally:
        print("Closing connection...")
        client_socket.close()
        print("Stopped.")

if __name__ == "__main__":
    start_client()
