
LIB=../target/debug/libexample.so

install: $(LIB)
	sudo install $(LIB) /usr/lib/zsh/5.9/zsh/zexample.so

$(LIB): src/lib.rs
	cargo build

.PHOENY: install
