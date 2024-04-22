package main

import (
	"crypto/tls"
	"errors"
	"fmt"
	"io"
	"net"
	"os"
	"time"
)

func exitWithError(err interface{}) {
	fmt.Println(err)
	os.Exit(-1)
}

var expectedHandshake = []byte{0x60, 0x60, 0xb0, 0x17}

// Do not print anything on stdout until listening!
func main() {
	var (
		listener       net.Listener
		err            error
	)
	address := "0.0.0.0:6666"
	certPath := "certs/server_thehost.pem"
	keyPath := "certs/server_thehost.key"
	minTlsMinorVer := 2
	maxTlsMinorVer := 2

	cert, err := tls.LoadX509KeyPair(certPath, keyPath)
	if err != nil {
		exitWithError(err)
	}

	config := tls.Config{
		GetCertificate: func(hello *tls.ClientHelloInfo) (*tls.Certificate, error) {
			return &cert, nil
		},
		MinVersion: 0x0300 | uint16(minTlsMinorVer+1),
		MaxVersion: 0x0300 | uint16(maxTlsMinorVer+1),
	}

	listener, err = tls.Listen("tcp", address, &config)
	if err != nil {
		exitWithError(err)
	}
	fmt.Printf("TLS, listening on %s with cert %s\n", address, certPath)
	defer listener.Close()

	for {
		conn, err := listener.Accept()
		if err != nil {
			exitWithError(err)
		}
		go func(conn net.Conn) {
			conn.SetReadDeadline(time.Now().Add(5 * time.Second))
			fmt.Printf("TLS, client connected from %s, waiting for Bolt handshake\n", conn.RemoteAddr())

			handshake := make([]byte, 4*5)
			_, err = io.ReadFull(conn, handshake)
			if err != nil {
				fmt.Println("Failed to receive Bolt handshake")
				exitWithError(err)
			}
			_, err = conn.Write([]byte{0x00, 0x00, 0x00, 0x00})
			if err != nil {
				fmt.Println("Failed to send Bolt handshake")
				exitWithError(err)
			}
			conn.Close()

			// Just check the signature
			for i, x := range expectedHandshake {
				if x != handshake[i] {
					exitWithError(errors.New("Bad Bolt handshake"))
				}
			}

			fmt.Println("Client connected with correct Bolt handshake")
		}(conn)
	}
}
