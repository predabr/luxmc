import DOMPurify from "dompurify";
export function sanitizeHtml(html: string): string {
		return DOMPurify.sanitize(html, {
			USE_PROFILES: { html: true },
			FORBID_TAGS: ["form", "input", "button", "iframe", "object", "embed"],
			FORBID_ATTR: ["style", "class", "srcdoc"]
		});
	}

