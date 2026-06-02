# 1.9.1 Release Supersession Note

The public `v1.9.0` tag existed before matching 1.9.x package-boundary release evidence was retained. That tag is treated as historical and superseded rather than rewritten.

This 1.9.1 release evidence is retained for the precision crate-boundary split after the workspace/package boundary split from the former `dpw4` crate into `precision-math` and `precision-cli`.

The prior retained 1.8.0 release evidence remains historical. The 1.9.1 evidence is the active retained release record for the crate-boundary split tree.

Related hardware evidence:
- `artifacts/hil_timing_dual/0008/` retains a BBB dual-board observer rerun using the existing retained 0007 context as scratch.
- Initial retained-context scratch rerun attempts failed with observer report timeout.
- After power cycling the boards, rerunning the retained 0007 context as scratch succeeded and produced the retained 0008 evidence package.
