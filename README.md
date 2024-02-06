# Trackers

## Syntax

* Application is the default operation.
    * `func(3.5)` is equivalent to `func 3.5`. Should be parsed as an identifier
        followed by a bracketed expression which is the first argument.

## Primitives
* Rolls. These are stored as a `(die, quantity)` pair.
    * `d20`
    * `8d8`
* Integers. These are stored as `u32`. Rolls can be coerced to integers by
    evaluating them and taking a some of their outcomes.
* Numbers. These are stored as `f64`. Integers can be coerced to numbers.
* Lists. These contain a collection of integers.
    * `[1, 2, 3, 4, 5]`
* Evaluated rolls. These are a `(roll, list)` pair. They can be coerced to a
    list by discarding the roll.
    * `(d20, [18])`
    * `(4d8, [2, 4, 6, 8])`

## Methods
Methods available in the global scope to use in expressions.
* Arithmetic: `+ - * / ^`, infix arithmetic operators. PEMDAS binding.
* `floor(decimal): integer`, mathematical floor. `floor 3.8 == 3`.
* `ceil(decimal): integer`, mathematical ceil. `ceil 1.2 == 2`.
* `dice(roll): integer`, size of roll dice. `dice 8d8 == 8`.
* `avg(roll): decimal`, average outcome of a roll. `avg 3d8 == 13.5`.

## Global Variables
* `?`, the output of the previous command.
* Player level, `LEVEL`, integer.
* Stats; `STRENGTH`, `DEXTERITY`, `CONSTITUTION`, `INTELLIGENCE`, `WISDOM`, `CHARISMA`.
* Modifiers; `STR`, `DEX`, `CON`, `INT`, `WIS`, `CHA`
    * These are calculated from stats like `STR = floor((STRENGTH - 10) / 2)`

## Examples
```
hd = d8
max_hp = dice(hd) + LEVEL * CON + (LEVEL - 1) * ceil(avg(hd))
```
