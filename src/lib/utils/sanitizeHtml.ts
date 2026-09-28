import DOMPurify from "dompurify";
export function sanitizeHtml(html: string): string {
		return DOMPurify.sanitize(html, {
			USE_PROFILES: { html: true },
			FORBID_TAGS: ["form", "input", "object", "embed"],
			ADD_TAGS: ["video", "source", "track", "audio"],
			ADD_ATTR: ["controls", "preload", "poster", "target", "rel", "data-video-id", "role", "tabindex"],
			FORBID_ATTR: ["srcdoc"]
		});
	}

