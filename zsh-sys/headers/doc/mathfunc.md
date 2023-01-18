# Zsh module math function definition
zsh allows you to define your own math function definitions. It doesn't mean they necessarily are used to do math, they are just used in math contexts.

The example module (zsh/example) defines a "sum" and a "length" condition.

A math string function receives as the argument everything inside the parenthesis, so `echo $((length(hello)))` evaluates to 5, while `echo $((length(  hello  )))` evaluates to 9.
