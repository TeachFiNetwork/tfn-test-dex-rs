<p align="center">
  <a href="https://teachfi.network/" target="blank"><img src="https://teachfi.network/teachfi-logo.svg" width="256" alt="TeachFi Logo" /><br/>Test DEX</a>
</p>
<br/>
<br/>
<br/>

# Description

This is a child contract of Platform SC. A separate instance is deployed for each platform subscriber.\
<br/>
<br/>
<br/>
## Endpoints

<br/>

```rust
createPair(
    base_token: TokenIdentifier,
    token: TokenIdentifier,
    decimals: u8,
    lp_fee: u64,
    owner_fee: u64,
)
```
>[!IMPORTANT]
>*Requirements:* state = active, base token should be in the allowed list.

>[!NOTE]
>Creates a new trading pair for the specified `token` on parity with `base_token` and with the specified fees. 
>The default pair state will be ActiveNoSwap, which means it will only be possible to add/remove liquidity, but not trade yet.

>[!WARNING]
>The transaction should have a 0.05 eGLD value, needed to issue the LP token for the newly created pair.
<br/>

```rust
setPairActive(id: usize)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = pair owner, pair liquidity > 0.

>[!NOTE]
>Activates trading for the pair specified by the `id` parameter.
<br/>

```rust
setPairActiveNoSwap(id: usize)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = pair owner.

>[!NOTE]
>Disables trading for the pair specified by the `id` parameter. Liquidity add/remove operations are still possible.
<br/>

```rust
setPairInactive(id: usize)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = pair owner.

>[!NOTE]
>Disables all operations on the pair specified by the `id` parameter.
<br/>

```rust
changePairFees(id: usize, new_lp_fee: u64, new_owner_fee: u64)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = pair owner.

>[!NOTE]
>Changes the trading fees of the pair specified by the `id` parameter. Example: for 0.75%, you need to send 75 to the SC.
<br/>

```rust
addBaseToken(token: TokenIdentifier)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = platform subscriber.

>[!NOTE]
>Adds a new base token, on parity with which can be created new pairs.
<br/>

```rust
removeBaseToken(token: TokenIdentifier)
```
>[!IMPORTANT]
>*Requirements:* state = active, caller = platform subscriber.

>[!NOTE]
>Removes the specified base token and new pairs can no longer be created on parity with it. Existing pairs are not affected.
<br/>

```rust
setStateActive()
```
>[!IMPORTANT]
*Requirements:* the caller must be the SC owner.

>[!NOTE]
>Sets the SC state as active.
<br/>

```rust
setStateInactive()
```
>[!IMPORTANT]
*Requirements:* the caller must be the SC owner.

>[!NOTE]
>Sets the SC state as inactive.
<br/>

```rust
setPlatformAddress(platform_sc: ManagedAddress)
```
>[!IMPORTANT]
>*Requirements:* caller = owner, platform should be empty.

>[!NOTE]
>Sets the Platform SC address and retrieves the governance token id from it.

<br/>

## View functions

```rust
getState() -> State
```
>Returns the state of the SC (Active or Inactive).
<br/>

```rust
getPlatformAddress() -> ManagedAddress
```
>Returns the Platform SC address if set.
<br/>

```rust
getBaseTokens() -> ManagedVec<TokenIdentifier>
```
>Returns the list of base tokens, on parity with which new trading pairs can be created.
<br/>

```rust
getPair(id: usize) -> Pair
```
>Returns the Pair object associated with the `id` parameter.
<br/>

```rust
getPairs() -> ManagedVec<Pair>
```
>Returns all trading pairs.
<br/>

```rust
getPairByTickers(token1: TokenIdentifier, token2: TokenIdentifier) -> Option<Pair>
```
>If a trading pair with the specified tokens is found, Some(pair) is returned and None otherwise.
<br/>

```rust
getPairByLpToken(lp_token: TokenIdentifier) -> Option<Pair>
```
>If a trading pair with the specified `lp_token` is found, Some(pair) is returned and None otherwise.

<br/>

## Custom types

```rust
pub enum State {
    Inactive,
    Active,
}
```

<br/>

```rust
pub enum PairState {
    Inactive,
    ActiveNoSwap,
    Active,
}
```

<br/>

```rust
pub struct Pair<M: ManagedTypeApi> {
    pub id: usize,
    pub owner: ManagedAddress<M>,
    pub state: PairState,
    pub token: TokenIdentifier<M>,
    pub decimals: u8,
    pub base_token: TokenIdentifier<M>,
    pub lp_token: TokenIdentifier<M>,
    pub lp_supply: BigUint<M>,
    pub lp_fee: u64,
    pub owner_fee: u64,
    pub liquidity_token: BigUint<M>,
    pub liquidity_base: BigUint<M>,
}
```
