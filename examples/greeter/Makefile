
LIB=../target/debug/libgreeter.so
TARGET_FOLDER=/usr/lib/zsh/5.9/
TARGET_NAME=rgreeter.so

install: $(LIB)
	sudo install $(LIB) $(TARGET_FOLDER)/$(TARGET_NAME)

uninstall: $(LIB)
	sudo rm -f $(TARGET_FOLDER)/$(TARGET_NAME)

$(LIB): src/lib.rs
	cargo build

.PHOENY: install uninstall
