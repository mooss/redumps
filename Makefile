.PHONY: build clean

build:
	go build -o redditprocessor main.go

clean:
	rm redditprocessor
