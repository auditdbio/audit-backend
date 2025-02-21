# Report Generator

---

### Usage:

1. Install dependencies with command `npm i`;
2. Add environment variables from .env.example file;
3. Run project build `npm run build`;
4. Start the app `npm run start`.

## API Documentation:

### Generate Report
Generates a PDF report based on the provided data.

### Endpoint
`POST /api/generate-report`

### Request Body

#### Main Fields

| Field        | Type | Required | Description |
|--------------|-----|--------------|-----------|
| project_name | string | Yes | Project name |
| profile_link | string | No | Profile link (default: "{PROTOCOL}://{FRONTEND}") from env |
| auditor_name | string | No | Auditor name |
| audit_link   | string | No | QR code link (default: "{PROTOCOL}://{FRONTEND}") from env |
| report_data  | array | Yes | Array of report blocks |

#### Block Types (report_data)

Each block in `report_data` must conform to one of the following types:

##### Base Block Structure:
| Field | Type | Required | Description |
|------|-----|--------------|-----------|
| type | string | Yes | Block type: "markdown", "plain_text", "issue_data", "statistics", "scope" |
| title | string | Yes | Block title |
| text | string | No | Text or markdown markup |
| subsections | array | No | Array of nested blocks (recursive structure) |

##### Issue Data Block (issue_data type):

```ts
{
    "type": "issue_data", // required
    "title": string, // required
    "text": string, // optional, markdown
    "feedback": string, // optional, markdown
    "issue_data": {
        "severity": "Critical" | "Major" | "Medium" | "Minor",
        "links": string[], // optional
        "category": string, // optional
        "status": "Fixed" | "WillNotFix",
        "file": string // optional
    }
}
```

##### Statistics Block (statistics type):

```ts
{
    "type": "statistics",
    "title": string,
    "statistics": {
        "fixed": {
            "critical": number,
            "major": number,
            "medium": number,
            "minor": number
        },
        "not_fixed": {
            "critical": number,
            "major": number,
            "medium": number,
            "minor": number
        }
    }
}
```

##### Scope Block (scope type):

```ts
{
    "type": "scope",
    "title": string,
    "links": string[] // array of URLs
}
```

### Response:

- **Content-Type**: `application/pdf`
- **Body**: Binary PDF file data
