Staking contract

Execute
    UpdateConfig : changes contract's owner
    UpdateConstants: changes main variables
    UpdateEnabled: Enables or disables the contract's working state
    Receive: Receives the verse token( simply send verse token to staking contract)
    CreateUnstake: Registers new unstake entry
    FetchUnstake: Withdraw some unstaked amount from th expired unstake entry
    MintVerse: VERSE token mint function
Query
    Config: Reads the basic config
    Staker: Gets the information of the special staker
    ListStakers: Lists all stakers info
    Unstaking: Lists a user's unstaking info
    Apys: Shows the apy list
    History: shows history of each user
