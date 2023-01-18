# Zsh module condition definitions
zsh allows you to define your own custom condition.

The example module (zsh/example) defines a "ex" and a "len" condition.

They are used as follows:
- `[[ -len a b ]]`
    - Returns 0 if the length of a equals the value of b
    - Ex: `[[ -len 'test' 4 ]]; # 0`
- `[[ a -ex b ]]`
    - Returns 0 if a concat b equals "example"
    - Ex: `[[ ex -ex ample ]]; # 0`
