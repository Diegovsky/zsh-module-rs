# How to Install
```bash
# build and install module into zsh module_path
make TARGET_FOLDER=$module_path
# unload first to use the latest module
if zmodload | grep rgreeter; then zmodload -u rgreeter; fi
# load it
zmodload rgreeter 
# run the new built-in greet command
greet
```
