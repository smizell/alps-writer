# ALPS Writer

ALPS writer is a tool for writing ALPS profiles in Markdown files.

- It uses the structure of the `profile` directory as the structure for the profile
- It uses the YAML frontmatter in the Markdown files as the attributes for the descriptors
- It uses the body of the Markdown files as the documentation for the descriptor
- It uses the name of the Markdown file as the ID of the descriptor
- It uses the `index.md` of a directory to define the definition for a descriptor with nested descriptors

Look at the `example/profile` directory for an example of how this works. It's structure is:

```
example/profile
├── collection
│   ├── index.md
│   └── nameSearch.md
├── contact
│   ├── email.md
│   ├── fullName.md
│   ├── index.md
│   ├── item.md
│   └── phone.md
└── index.md
```

And the output of `alps-writer profile ./example/profile` is:

```js
{
  "alps": {
    "doc": {
      "format": "markdown",
      "value": "A contact list."
    },
    "link": [
      {
        "rel": "help",
        "href": "http://example.org/help/contacts.html"
      }
    ],
    "descriptor": [
      {
        "id": "index",
        "doc": {
          "format": "markdown",
          "value": ""
        },
        "type": "semantic",
        "descriptor": [
          {
            "id": "email",
            "doc": {
              "format": "markdown",
              "value": ""
            },
            "type": "semantic",
            "descriptor": []
          },
          {
            "id": "fullName",
            "doc": {
              "format": "markdown",
              "value": ""
            },
            "type": "semantic",
            "descriptor": []
          },
          {
            "id": "item",
            "doc": {
              "format": "markdown",
              "value": "A link to an individual contact."
            },
            "type": "safe",
            "descriptor": []
          },
          {
            "id": "phone",
            "doc": {
              "format": "markdown",
              "value": ""
            },
            "type": "semantic",
            "descriptor": []
          }
        ]
      },
      {
        "id": "index",
        "doc": {
          "format": "markdown",
          "value": "A simple link/form for getting a list of contacts."
        },
        "type": "safe",
        "descriptor": [
          {
            "id": "nameSearch",
            "doc": {
              "format": "markdown",
              "value": "Input for a search form."
            },
            "type": "semantic",
            "descriptor": []
          }
        ]
      }
    ]
  }
}
```

## Usage

Use the help for information on how to use the CLI tool.

```sh
alps-writer --help
```
