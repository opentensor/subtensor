# Testing

Typescript tests are run with [Moonwall](https://github.com/Moonsong-Labs/moonwall). To run these you will need to have pnpm installed:

```
# Use the correct Node version
nvm use

# Install moonwall
sudo npm i -g pnpm

# Change directory to test
cd ts-tests

# Install dependencies
pnpm i

# Run manual seal dev tests
pnpm moonwall test dev

# Run zombienet tests
sudo pnpm moonwall test zombienet

# If you have MacOS, you might need to run zombinet test with sudo, because tmp folder
sudo sudo pnpm moonwall test zombienet

# Run smoke tests
sudo pnpm moonwall test smoke_mainnet
```

Moonwall lets you also run the testing environment without performing any tests on it, as a method for you to manually test certain things:

```
# Dev tests in run mode
sudo pnpm moonwall run dev

# Zombinet test with run mode
sudo pnpm moonwall run zombienet
```
