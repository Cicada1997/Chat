package main

import (
	"errors"
	"bufio"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"os/signal"
	"syscall"
)

type Packet map[string]map[string]interface{}

type LoginPayload struct {
	Login struct {
		Username string `json:"username"`
		Password string `json:"password"`
	} `json:"Login"`
}

type SendMessagePayload struct {
	SendMessage struct {
		Content   string `json:"content"`
		ChannelID uint32 `json:"channel_id"`
	} `json:"SendMessage"`
}

type DisconnectPayload struct {
	Disconnect struct {
		AuthorID  uint32 `json:"author_id"`
		Content   string `json:"content"`
		ChannelID uint32 `json:"channel_id"`
	} `json:"Disconnect"`
}

func sendJSON(conn net.Conn, v interface{}) error {
	b, err := json.Marshal(v)
	if err != nil {
		return err
	}
	_, err = fmt.Fprintln(conn, string(b))
	return err
}

func listen(conn net.Conn, c chan os.Signal) {
	sc := bufio.NewScanner(conn)
	for sc.Scan() {
		var packet Packet
		err := json.Unmarshal([]byte(sc.Text()), &packet)
		if err != nil {
			fmt.Println("Malformed packet: %v", packet)
		}

		if pack := packet["Disconnect"]; pack != nil {
			fmt.Printf("Disconnected for reason: %v\n", pack["reason"])
			<-c
		}

		if pack := packet["NewMessage"]; pack != nil {
			dname := pack["username"]
			if dname == nil {
				dname = pack["author_id"]
			}
			fmt.Printf("%v: %v\n", dname, pack["content"])
		}
	}
	if err := sc.Err(); err != nil {
		fmt.Println("read error:", err)
	}
}

func login(conn net.Conn) error {
	in := bufio.NewScanner(os.Stdin)
	for true {
		login := LoginPayload{}

		fmt.Print("Enter kattauth username: ")
		in.Scan()
		login.Login.Username = in.Text()

		fmt.Printf("Enter password for %v: ", login.Login.Username)
		in.Scan()
		login.Login.Password = in.Text()


		if login.Login.Password == "exit" || login.Login.Password == "quit" || login.Login.Username == "quit" || login.Login.Username == "quit" {
			return errors.New("initaited exit")
		}
		
		if err := sendJSON(conn, login); err != nil {
			fmt.Println("send login error:", err)
			continue
		}

		return nil
	}

	return errors.New("unreachable, contact the devs")
}

func main() {
	conn, err := net.Dial("tcp", "cicada.kattmys.se:1997")
	if err != nil { panic(err) }
	defer conn.Close()

	c := make(chan os.Signal, 1)
	signal.Notify(c, os.Interrupt, syscall.SIGTERM)

	go listen(conn, c)
	if err := login(conn); err != nil {
		if fmt.Sprintf("%v", err) == "initiated exit" {
			return
		}

		fmt.Println(err)
	}

	go func() {
		<-c
		disc := DisconnectPayload{}
		disc.Disconnect.AuthorID = 0
		disc.Disconnect.Content = "client disconnecting"
		disc.Disconnect.ChannelID = 1
		_ = sendJSON(conn, disc)
		conn.Close()
		os.Exit(0)
	}()

	in := bufio.NewScanner(os.Stdin)
	for in.Scan() {
		text := in.Text()
		if text == "" { continue }
		if text == "quit" || text == "exit" {
			break
		}

		msg := SendMessagePayload{}
		msg.SendMessage.Content = text
		msg.SendMessage.ChannelID = 1
		if err := sendJSON(conn, msg); err != nil {
			fmt.Println("send error:", err)
			break
		}
	}
	if err := in.Err(); err != nil {
		fmt.Println("stdin error:", err)
	}
}

