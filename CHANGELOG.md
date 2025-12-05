## v0.2.0

- Increased slots reward-selection hitbox size vertically (1 tile -> full column) making it considerably easier to click ([ebf4fc0](https://github.com/thehuglet/term-slots-rs/commit/ebf4fc074e5f27316b013e0fcae559299f889ab1))
- Added multi-column card selection from slots if their ranks match ([e2d53bd](https://github.com/thehuglet/term-slots-rs/commit/e2d53bd0415a3d5bbf29c1390204823f00d05755))

## v0.3.0

- Changed spin cost scaling formula to follow a linear curve instead of an exponential one
- Cards can now be picked from slots when cards are present on the table
- Added right click support to slots card picking
- Hovered and matching slots cards will now be highlighted in red when hand has no space in it
- Attempting to pick a slots card will now trigger a red flash warning on cards in hand if there is no space for it
