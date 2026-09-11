export const fadeIn = {
  duration: 200,
  css: (t: number) => `opacity: ${t}`
};

export const slideUp = {
  duration: 200,
  css: (t: number) => `transform: translateY(${(1 - t) * 10}px); opacity: ${t}`
};

export const scaleIn = {
  duration: 150,
  css: (t: number) => `transform: scale(${0.95 + t * 0.05}); opacity: ${t}`
};

export const blurIn = {
  duration: 200,
  css: (t: number) => `filter: blur(${(1 - t) * 4}px); opacity: ${t}`
};
