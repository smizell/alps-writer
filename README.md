# ALPS Writer

ALPS writer is a tool for writing ALPS profiles in Markdown files.

- It uses the structure of the `profile` directory as the structure for the profile
- It uses the YAML frontmatter in the Markdown files as the attributes for the descriptors
- It uses the body of the Markdown files as the documentation for the descriptor
- It uses the name of the Markdown file as the ID of the descriptor
- It overwrites the ID with the ID in the frontmatter if it exists
- It ignores the ID if the file name starts with an underscore
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
    "version": "1.0",
    "descriptor": [
      {
        "id": "contact",
        "descriptor": [
          {
            "id": "email"
          },
          {
            "id": "fullName"
          },
          {
            "id": "item",
            "doc": {
              "format": "markdown",
              "value": "A link to an individual contact."
            },
            "type": "safe"
          },
          {
            "id": "phone"
          }
        ]
      },
      {
        "id": "collection",
        "rt": "#contact",
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
            }
          }
        ]
      }
    ],
    "doc": {
      "format": "markdown",
      "value": "A contact list."
    },
    "link": [
      {
        "rel": "help",
        "href": "http://example.org/help/contacts.html"
      }
    ]
  }
}

```

## Install

Visit the latest [releases page](https://github.com/smizell/alps-writer/releases/) and download the latest for your OS. You do not need to install anything—you can directly run the binary.

## Usage

Use the help for information on how to use the CLI tool.

```sh
alps-writer --help
```
