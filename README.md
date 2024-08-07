# TODO

* Add line and column information to tokens, eventually AST nodes.
* Coin implementation bi-recurses infinitely if trying to break a larger coin.
* `d` as in roll literals could just be an operator creating a roll from two
    ints.

# Trackers

## Syntax

### Grammar

```
expr := term { binary term }
term :=
    factor
    | call
    | list 
    | ( expr )
    | unary-prefix term
    | term unary-postfix
    | if expr then expr { else expr }
binary := + | - | * | / | ^ | k | =
call := identifier ( expr { , expr } )
list := [ expr { , expr } ]
unary-postfix := a | d | s | k
unary-prefix := -
factor := roll | number | identifier
roll := /[0-9]*d[0-9]+/ 
number := /[0-9]+(.[0-9]+)?/
identifier := /[a-zA-Z][a-zA-Z0-9]*/
```

### Notes

* Declare functions with `:=`, declare constants with `=`.
    * A function is stored as a parsed expression, and evaluated when
        referenced.
    * A constant is evaluated immediately and the resultant value is stored in
        the variable specified.

```
> sub(a, b) := a - b
sub(a, b) = a - b
> sixteen = sub(4 * 5, 4)
sixteen = 16
> sixteen + 3
19
> 2d4 + sixteen
2d4    Rolls: 1, 3    Total: 4
2d4 + sixteen    Total: 20
```

## Primitives
* Rolls. These are stored as a `(die, quantity)` pair.
    * `d20`
    * `8d8`
* Integers. These are stored as `u32`. Rolls can be coerced to integers by
    evaluating them and taking a some of their outcomes.
* Numbers. These are stored as `f64`. Integers can be coerced to numbers.
* Strings. These are stored as rust `String`s. They are written `"string"`.
* Lists. These contain a collection of other values.
    * `[1, 2, 3, 4, 5]`
    * `["a", "b", 2]`
    * `[["a"], [1]]`
    * `[d4, d6, d8, d10, d12]`
* Evaluated rolls. These are a `(roll, list)` pair. They can be coerced to a
    list by discarding the roll.
    * `(d20, [18])`
    * `(4d8, [2, 4, 6, 8])`

## Built Ins
Functions available in the global scope to use in expressions.
* Arithmetic: `+ - * / ^`, infix arithmetic operators. PEMDAS binding.
* `floor(decimal): integer`, mathematical floor. `floor(3.8) == 3`.
* `ceil(decimal): integer`, mathematical ceil. `ceil(1.2) == 2`.
* `quantity(roll): integer`, number of roll dice. `quantity(10d4) == 4`.
* `dice(roll): integer`, size of roll dice. `dice(6d8) == 8`.
* `avg(roll): decimal`, average outcome of a roll. `avg(3d8) == 13.5`.

## Global Variables
* `?`, the output of the previous command.
* Player level, `LEVEL`, integer.
    * Used to calculate `PROF`, proficiency bonus and `EXPT`, expertise bonus.
* Stats; `STRENGTH`, `DEXTERITY`, `CONSTITUTION`, `INTELLIGENCE`, `WISDOM`, `CHARISMA`.
* Modifiers; `STR`, `DEX`, `CON`, `INT`, `WIS`, `CHA`
    * Calculated from stats using `modifier(stat) = floor((stat - 10) / 2)`

## Examples
```
hd = d8
max_hp = dice(hd) + LEVEL * CON + (LEVEL - 1) * ceil(avg(hd))
```
