def farewell(name, shout=False):
    text = f"bye {name.strip()}"
    return text.upper() if shout else text
