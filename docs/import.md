The Monzo API is broken. They provide a 'Pot' facility for separating money with an account. But for reasons known only to them, any transactions made
from a pot are not included in transaction data. If you use pots, and you want to use this, you'll need to manually add those transactions.

Create a `.csv` file for each pot you want to import with the following fields:

```text
 date,description,amount,local_currency,local_amount,category
 2024-04-14,PATH TAPP PAYGO CP NEW JERSEY USA,-0.8,USD,-1.0,Transport
```

Then execute the following command:

```shell
> monzo-beancount import
```

This will create a `.beancount` file for each `.csv` file, place them in the `include directory`, and include them in the main file.
