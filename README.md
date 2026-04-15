# CicadaChat
**An rust chat app backend connected to the [Cat Mouse Network](https://kattmys.se/).**

## Server API
To communicate with the server you need to know how the protocol works. It is fairly simple: it uses a tcp stream to
send packets decoded into json between the client and server. 

### The First Packet
At the beginning of a connection the server will go into and auth mode where it only accepts auth packets, else it
failes. Here you need to send one of the following JSON packets:

```json
{
    "Login": {
        "username": String,
        "password": String
    }
}

// or for token login:
{
    "TokenLogin": {
        "token": String
    }
}
```

### Transmitting
To send a message after establishing a connection and logging in, you send a packet like the following:
```json
{
    "SendMessage": {
        "content": String,
        "channel_id": u32 // OBS: unimplemented.
    }
}

### Receiving
As soon as you have authenticated yourself, you will begin to receive JSON packets from the server. Here are the
different kinds:


```json
{
    "NewMessage": {
        "author_id": u32,
        "content": String,
        "channel_id": u32 // OBS: unimplemented.
    }
}
```
This is broadcasted to all users upon a recieved message from a client.

```json
{
    "Disconnect": {
        "author_id": u32,
        "content": String,
        "channel_id": u32 // OBS: unimplemented.
    }
}
```
This is sent to you as you are being disconnected for reason mentioned. It will also be broadcasted upon server
shutdown.
