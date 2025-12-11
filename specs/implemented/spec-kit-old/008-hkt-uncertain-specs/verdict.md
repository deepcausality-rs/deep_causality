# Verdict

The uncertain crate has been transitioned from the previously brittle compute node to the much more robust const tree crate for the internal AST data structure. 

However, the implementation of higher kinds of types has been proven exceptionally difficult because the type T in the uncertain type would require a trait bond, which the higher kind of type traits do not allow. Therefore, it was deemed sensible to exclude the higher kind type trait implementation for the time being. 