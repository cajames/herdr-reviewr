from farewell import farewell


def test_shout():
    assert farewell(" ada ", shout=True) == "BYE ADA"
