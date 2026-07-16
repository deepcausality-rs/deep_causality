import Lib.Thing

/-- Type-checks against the *prebuilt* `Lib.Thing` olean supplied by the
`lean_library` dep — proving consumption needs no source re-share or recompile. -/
def Consumer.y : Nat := Lib.thing + 1

example : Consumer.y = 42 := rfl
