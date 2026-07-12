read @AGENTS.md

plz review PR: GH link ...

Only review Rustcode, Rust tests, dosctring, crate configutrations and crate level README.md files. 

Ignore everything under the openspec folder.
Ignore everything MIRI related. 

For larger reviews (> 50 files) or multiple crates, you may start multiple review agents that
use these instructions plus your guidance. 

Look out for
- Oblivious bugs
- Correctness bugs 
- Logic bugs 
- Tests that falsey assert correct results by tautlogy
- Tests that would silently fail or fail to report a failure 
- Tests that do not cover the error path or any exception paths
- Poor performance choices in the implementation that can be replaced with a better one that delivers equivalent results while being measurably faster. 
- Abstraction leaks in the implementation
- Serious code quality issues. 
- Any major or minor security issues.

If you find any issues, diagnose each issue,identify  the root cause,  and derive a proper solution for each issue.

Then report all consolidated findings and let me decide which to resolve.

Note, if there are no major issues found, say so.
