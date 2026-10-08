# Resource.cfg load-order support

The Manager reads Resource.cfg to evaluate load order and coverage. By default it does not modify the file.

## Optional organization-time extension

A user can explicitly check **Extend Resource.cfg** in Organization Plan for roots within
`Mods`, `Mods/Packages` or `Mods/Overrides`. This preserves the full suggested
category folders rather than flattening them to old configuration depth.

Before moving a single CC package, the Manager:
- previews every new PackedFile depth rule and preserves the existing priorities;
- refuses advanced directives, multiple priority tiers for the same loading branch,
  unsupported settings and paths outside loading branches instead of guessing;
- confirms the proposed rules and exact SHA-256 of the original configuration;
- saves the original Resource.cfg to a unique `Resource.cfg.s3cc-backup-*` sibling;
- installs the expanded rules without rewriting existing lines;
- checks that every selected destination is loadable, rolling back a failed change;
- records both hashes and the backup location in the organization restore manifest.

During normal Restore, the original Resource.cfg is recovered **only when** its
backup is unchanged, the currently installed cfg matches the recorded update, and
all existing Packages/Overrides are covered by the original rules. If any check
fails, Restore keeps the updated configuration and warns the user rather than
making packages invisible. A backup remains for manual recovery.

Ordinary external/staging libraries are never offered the Resource.cfg option.

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
