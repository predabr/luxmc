const reduced = matchMedia('(prefers-reduced-motion: reduce)');

const targets = () => {
	const nodes = document.querySelectorAll(
		'main section > .container > h2, main section > .container > .section-tag, .project-card, .plan-card, .download-card, .feature-strip article, .party-card, .hero-copy > *, .hero-capture'
	);
	return [...nodes].filter((node) => !node.classList.contains('js-enter'));
};

function prepare() {
	if (reduced.matches) return;
	for (const node of targets()) {
		if (node.getBoundingClientRect().top < innerHeight * 0.92) {
			node.classList.add('js-enter', 'in-view');
		} else {
			node.classList.add('js-enter');
			observer.observe(node);
		}
	}
}

const observer = new IntersectionObserver(
	(entries) => {
		for (const entry of entries) {
			if (!entry.isIntersecting) continue;
			entry.target.classList.add('in-view');
			observer.unobserve(entry.target);
		}
	},
	{ rootMargin: '0px 0px -8% 0px', threshold: 0.08 }
);

if (document.readyState === 'loading') {
	document.addEventListener('DOMContentLoaded', prepare, { once: true });
} else {
	prepare();
}

// Conteúdo montado depois (catálogo, planos, contas) também entra suave.
const mutation = new MutationObserver(() => {
	if (reduced.matches) return;
	for (const node of targets()) {
		if (node.getBoundingClientRect().top < innerHeight * 0.92) node.classList.add('js-enter', 'in-view');
		else {
			node.classList.add('js-enter');
			observer.observe(node);
		}
	}
});
mutation.observe(document.body, { childList: true, subtree: true });

reduced.addEventListener('change', () => {
	for (const node of document.querySelectorAll('.js-enter')) node.classList.toggle('in-view', true);
});
