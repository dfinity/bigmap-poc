import default_ic, { HttpAgent, Principal, generateKeyPair, makeNonceTransform, makeExpiryTransform, makeAuthTransform } from "@dfinity/agent"

const keyPair = generateKeyPair()
const agent = new HttpAgent({
  principal: Principal.selfAuthenticating(keyPair.publicKey),
})
agent.addTransform(makeNonceTransform())
agent.addTransform(makeExpiryTransform(5 * 60 * 1000));
agent.setAuthTransform(makeAuthTransform(keyPair))

const ic = { ...default_ic, agent }
window.ic = ic

export { ic }
