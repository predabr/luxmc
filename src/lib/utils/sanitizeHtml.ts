import DOMPurify from "dompurify";
export function sanitizeHtml(html: string): string {
		return DOMPurify.sanitize(html, {
			USE_PROFILES: { html: true },
			FORBID_TAGS: ["form", "input", "button", "iframe", "object", "embed", "video", "audio", "source", "track"],
			FORBID_ATTR: ["style", "srcdoc"]
		});
	}

