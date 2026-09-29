# Resource.cfg load-order support

The Organizer reads Resource.cfg only to add evidence to conflict analysis.

It does not modify Resource.cfg.

## Supported commands

Current parser support:

```text
Priority <signed integer>
PackedFile <path pattern>
```

Other lines are preserved conceptually as unsupported/ignored for this milestone.

## Matching

PackedFile patterns are matched case-insensitively against the package path relative to the Resource.cfg directory.

Each `*` matches one path component's text, not multiple directory levels.

Examples:

```text
PackedFile Packages/*.package
PackedFile Packages/*/*.package
PackedFile Packages/*/*/*.package
```

A package can match more than one rule. The highest matching Priority is retained.

## Safety

The Organizer only says one package has higher configured priority when both packages matched rules and their numeric priorities differ.

It does not guess a winner for equal priorities.
