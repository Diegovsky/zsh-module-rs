
LIB=../target/debug/libgreeter.so

install: $(LIB)
	sudo install $(LIB) /usr/lib/zsh/5.9/zsh/rgreeter.so

uninstall: $(LIB)
	rm -f /usr/lib/zsh/5.9/zsh/rgreeter.so

$(LIB): src/lib.rs
	cargo build

.PHOENY: install uninstall
