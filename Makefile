all: compile

compile:
	cargo b --release

test: compile
	mkdir -p test
	rm -r test
	mkdir -p test
	cp -r chats/* ./test/
