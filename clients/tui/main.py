import socket
import threading
import json
import traceback

from dotenv import load_dotenv
import os

from typing import Dict

running = True

def handle_conn(sock: socket.socket):
    global running
    reader = sock.makefile("r", encoding="utf-8")

    while True:
        try:
            line = reader.readline()

            try: json_packet: Dict = json.loads(line)
            except json.JSONDecodeError as e:
                print(f"\r[ ERROR ] Unable to parse packet: {e}\n> ", end="", flush=True)
                continue

            match list(json_packet.keys())[0]:
                case "NewMessage":
                    msg = json_packet["NewMessage"]
                    name = msg["username"]
                    if not name: name = msg["author_id"]
                    print(f"\r{ name }: { msg["content"] }\n> ", end="", flush=True)
                
                case "Disconnect":
                    disconn_msg = json_packet["Disconnect"]
                    print(f"\rYou have been disconnected from the server. reason: {disconn_msg["reason"]}")
                    print("\n\tPress enter to exit...")
                    running = False
                    break

                case _:
                    print(f"\rCould not parse packet:\n{ json.dumps(json_packet) }\n> ", end="", flush=True)
                    continue

        except Exception as e:
            print(traceback.format_exc())

MESSAGE = lambda content: json.dumps({
        "SendMessage": {
            "content": content,
            "channel_id": 0,
        }
    }) + "\n"

LOGIN = lambda username, password: json.dumps({
    "Login": {
        "username": username,
        "password": password,
        }
    }) + "\n"
TOKEN_LOGIN = lambda token: json.dumps({
    "TokenLogin": {
        "token": token,
        }
    }) + "\n"

DISCONNECT = lambda: json.dumps("Disconnect") + "\n"

def login(client):
    load_dotenv()
    token = os.getenv("KATTAUTH_TOKEN")

    if token:
        client.sendall(TOKEN_LOGIN(token).encode("utf-8"))

    else:
        username = input("\rEnter kattauth username: ")
        password = input("\rEnter kattauth password: ")

        client.sendall(LOGIN(username, password).encode("utf-8"))

def start_client():
    host = "127.0.0.1"
    port = 19975
    # host = "cicada.kattmys.se"
    # port = 1997

    client_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    try:
        client_socket.connect((host, port))
        print(f"--- Connected to CicadaChat at {host}:{port} ---")

    except ConnectionRefusedError:
        print(f"Connection to {host}:{port} failed.")
        return

    threading.Thread(target=handle_conn, args=(client_socket,), daemon=True).start()

    login(client_socket)

    try:
        while running:
            msg = input("> ")
            if msg.lower() in ["exit", "quit"]:
                break
            
            client_socket.sendall(MESSAGE(msg).encode("utf-8"))

    except KeyboardInterrupt:
        client_socket.sendall(DISCONNECT().encode("utf-8"))
        pass

    finally:
        print("Closing connection...")
        client_socket.close()
        print("Stopped.")

if __name__ == "__main__":
    start_client()
