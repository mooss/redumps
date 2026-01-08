.PHONY: build clean

build:
	go build -o redumps main.go

clean:
	rm redumps
