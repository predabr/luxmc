let feature = $state('');
let requiredPlan = $state<'plus' | 'pro'>('plus');
export const planPrompt = {
    get feature() { return feature; },
    get requiredPlan() { return requiredPlan; },
    open(value: string, minimum: 'plus' | 'pro' = 'plus') { feature = value; requiredPlan = minimum; },
    close() { feature = ''; }
};
