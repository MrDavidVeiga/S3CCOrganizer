# Resource.cfg load-order support

The Organizer reads Resource.cfg only to add evidence to conflict analysis.

It does not modify Resource.cfg.

## Supported commands

Current parser support:

```text
Priority <signed integer>
PackedFile <path pattern>
```

DirectoryFiles is ignored for package precedence and reported as informational.

Advanced traversal/conditional directives currently not evaluated are:

```text
Scan
Select
End
StopScan
```

When any of these directives is present, the parser marks Resource.cfg precedence as **not reliable for automatic winner inference** and surfaces a warning in the Conflicts UI.

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

The Organizer only says one package has higher configured priority when:

- both packages matched supported PackedFile rules;
- their numeric priorities differ;
- the Resource.cfg does not use unsupported advanced traversal/conditional directives.

Higher Priority values take precedence over lower values for supported rules. The tool does not guess a winner for equal priorities or when advanced directives make the simplified model incomplete.
