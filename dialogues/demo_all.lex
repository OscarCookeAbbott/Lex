@Oscar
full_name: Oscar Cooke-Abbott
age: 26

$example_text: This is some text
$example_boolean: true
$example_number: 123.456
$example_array: [This is an entry, This is also an entry]

!example_function
!example_functions_args(arg_1=Some value, arg_2=Other value)
!example_function_text: "Default return value"


# Intro

// This is a comment.
/// This is a logged comment.
//? This is a logged warning.
//! This is a logged error.


This is an example of basic text.


This is an example of **rich** *text*.


This is an example of a variable reference: {$example_boolean}.
This is an example of a character property reference: {@Oscar.age}.


// This is an example of overriding a variable value
$example_boolean = false


// These are examples of calling functions
!example_function
!example_function_args(New arg value)
$example_text = !example_function_text


@Oscar: This is an example of identified spoken dialogue.
Other Oscar: This is an example of anonymous spoken dialogue.


- This is a response
    $example_number = 654.321

    This is branching dialogue.

- This is also a response
    - This is a nested response
        With its own nested dialogue.


| This is a single page despite having line breaks:
|
| - Wow!
| - This **is** more readable!


[mood=info]
This is an example of annotated dialogue.


[if=$example_boolean]
This is a conditional block.


~ IF $example_number > 18

This is branching dialogue.

Across multiple pages.

~ ELSE

This is an alternate branch.

~


~ REPEAT 3

This will happen three times.

~


~ WHILE $example_number < 999_999

This will repeat until the condition is satisfied.

$example_number += 100_000

~


~ EACH $example_array as $example_entry

This will be repeated **on a single page** for each entry: {$example_entry}

~


// This is a section jump
=> #Outro

// This is a section end jump, it ends the current section
=> END

// This is a section bounce-jump, once the jumped section its dialogue will resume from here
=><= #Outro

// This is a terminal jump, it forcefully terminates the current dialogue
=> TERMINATE
