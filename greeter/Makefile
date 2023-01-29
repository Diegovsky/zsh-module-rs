
LIB=../target/debug/libgreeter.so
TARGET_FOLDER=/usr/lib/zsh/5.9/

install: $(LIB)
	sudo install $(LIB) 

uninstall: $(LIB)
	sudo rm -f /usr/lib/zsh/5.9/zsh/rgreeter.so

$(LIB): src/lib.rs
	cargo build

.PHOENY: install uninstall
