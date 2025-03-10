const BigNumber = require('bignumber.js');
const {
  deployAppWithMockChannels,
  addressBytes,
  ChannelId,
} = require("./helpers");
require("chai")
  .use(require("chai-as-promised"))
  .use(require("chai-bignumber")(BigNumber))
  .should();

const MockOutboundChannel = artifacts.require("MockOutboundChannel");

const ScaleCodec = artifacts.require("ScaleCodec");
const ERC721App = artifacts.require("ERC721App");
const TestToken = artifacts.require("TestToken721");

const approveToken = (tokenContract, tokenId, app, account) => {
  return tokenContract.approve(app.address, tokenId, { from: account })
}

const lockupToken = (app, tokenContract, tokenId, sender, recipient, channel) => {
  return app.lock(
    tokenContract.address,
    tokenId.toString(),
    addressBytes(recipient),
    channel,
    {
      from: sender,
      value: 0
    }
  )
}

contract("ERC721App", function (accounts) {
  // Accounts
  const owner = accounts[0];
  const inboundChannel = accounts[1];
  const userOne = accounts[2];
  const userTwo = accounts[3];
  const tokenId = 1;
  const anotherTokenId = 2;

  // Constants
  const POLKADOT_ACCOUNT_ID = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"

  before(async function () {
    const codec = await ScaleCodec.new();
    ERC721App.link(codec);
  });

  describe("lock", function () {
    beforeEach(async function () {
      let outboundChannel = await MockOutboundChannel.new()
      this.app = await deployAppWithMockChannels(owner, [inboundChannel, outboundChannel.address], ERC721App);
      this.symbol = "TEST";
      this.token = await TestToken.new("Test Token", this.symbol);

      await this.token.mintWithTokenURI(userOne, tokenId, "http://testuri.com/nft.json", {
        from: owner
      }).should.be.fulfilled;

      await this.token.mint(userOne, anotherTokenId, {
        from: owner
      }).should.be.fulfilled;
    });

    it("should lock token with tokenURI metadata", async function () {
      await approveToken(this.token, tokenId, this.app, userOne)
        .should.be.fulfilled;

      let tx = await lockupToken(this.app, this.token, tokenId, userOne, POLKADOT_ACCOUNT_ID, ChannelId.Basic)
        .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = tx.logs.find(
        e => e.event === "Locked"
      );

      event.args.tokenContract.should.be.equal(this.token.address);
      BigNumber(event.args.tokenId).should.be.bignumber.equal(tokenId);
      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ACCOUNT_ID);

      let newOwner = await this.token.ownerOf(tokenId);
      newOwner.should.be.equal(this.app.address);
    });

    it("should lock token without tokenURI", async function () {
      await approveToken(this.token, anotherTokenId, this.app, userOne)
        .should.be.fulfilled;

      let tx = await lockupToken(this.app, this.token, anotherTokenId, userOne, POLKADOT_ACCOUNT_ID, ChannelId.Basic)
        .should.be.fulfilled;

      // Confirm app event emitted with expected values
      const event = tx.logs.find(
        e => e.event === "Locked"
      );

      event.args.tokenContract.should.be.equal(this.token.address);
      BigNumber(event.args.tokenId).should.be.bignumber.equal(anotherTokenId);
      event.args.sender.should.be.equal(userOne);
      event.args.recipient.should.be.equal(POLKADOT_ACCOUNT_ID);

      let newOwner = await this.token.ownerOf(anotherTokenId);
      newOwner.should.be.equal(this.app.address);
    });

    it("should fail to lock if not approved", async function () {
      await lockupToken(this.app, this.token, anotherTokenId, userOne, POLKADOT_ACCOUNT_ID, ChannelId.Basic)
        .should.be.rejectedWith(/transfer caller is not owner nor approved/);
    });

    it("should fail to lock if not approved or owner", async function () {
      await approveToken(this.token, tokenId, this.app, userOne)
        .should.be.fulfilled;

      // note that now userTwo tries to lock the tokens, who is not the owner and not approved by userOne
      await lockupToken(this.app, this.token, anotherTokenId, userTwo, POLKADOT_ACCOUNT_ID, ChannelId.Basic)
        .should.be.rejectedWith(/transfer caller is not owner nor approved/);
    });

    it("should fail to lock if invalid token contract", async function () {
      await approveToken(this.token, tokenId, this.app, userOne)
        .should.be.fulfilled;

      await lockupToken(this.app, { address: "0xfafafafafafafafafafafafafafafafafafafafa" }, anotherTokenId, userOne, POLKADOT_ACCOUNT_ID, ChannelId.Basic)
        .should.be.rejectedWith(/revert/);
    });

    it("should fail to lock if invalid token id", async function () {
      await approveToken(this.token, tokenId, this.app, userOne)
        .should.be.fulfilled;

      await lockupToken(this.app, this.token, 1337, userOne, POLKADOT_ACCOUNT_ID, ChannelId.Basic)
        .should.be.rejectedWith(/operator query for nonexistent token/);
    });
  });

  describe("unlock", function () {
    beforeEach(async function () {
      let outboundChannel = await MockOutboundChannel.new()
      this.app = await deployAppWithMockChannels(owner, [inboundChannel, outboundChannel.address], ERC721App);
      this.symbol = "TEST";
      this.token = await TestToken.new("Test Token", this.symbol);

      await this.token.mintWithTokenURI(userOne, tokenId, "http://testuri.com/nft.json", {
        from: owner
      }).should.be.fulfilled;
    });

    it("should unlock funds", async function () {
      await approveToken(this.token, tokenId, this.app, userOne)
        .should.be.fulfilled;

      await lockupToken(this.app, this.token, tokenId, userOne, POLKADOT_ACCOUNT_ID, ChannelId.Basic)
        .should.be.fulfilled;

      let tx = await this.app.unlock(
        this.token.address,
        tokenId.toString(),
        addressBytes(POLKADOT_ACCOUNT_ID),
        userTwo,
        {
          from: inboundChannel
        }
      ).should.be.fulfilled;

      const event = tx.logs.find(e => e.event === "Unlocked");

      event.args.tokenContract.should.be.equal(this.token.address);
      BigNumber(event.args.tokenId).should.be.bignumber.equal(tokenId);
      event.args.sender.should.be.equal(POLKADOT_ACCOUNT_ID);
      event.args.recipient.should.be.equal(userTwo);
    });

    it("should fail to unlock if not locked", async function () {
      await approveToken(this.token, tokenId, this.app, userOne)
        .should.be.fulfilled;

      await this.app.unlock(
        this.token.address,
        tokenId.toString(),
        addressBytes(POLKADOT_ACCOUNT_ID),
        userTwo,
        {
          from: inboundChannel
        }
      ).should.be.rejectedWith(/transfer of token that is not own/);
    });

    it("should fail to unlock if not channel", async function () {
      await approveToken(this.token, tokenId, this.app, userOne)
        .should.be.fulfilled;

      await lockupToken(this.app, this.token, tokenId, userOne, POLKADOT_ACCOUNT_ID, ChannelId.Basic)
        .should.be.fulfilled;

      await this.app.unlock(
        this.token.address,
        tokenId.toString(),
        addressBytes(POLKADOT_ACCOUNT_ID),
        userTwo,
        {
          from: userTwo
        }
      ).should.be.rejectedWith(/Caller is not an inbound channel/);;
    });
  });
})
