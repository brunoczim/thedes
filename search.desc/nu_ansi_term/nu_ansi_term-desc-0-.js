searchState.loadedDescShard("nu_ansi_term", 0, "This is a library for controlling colors and formatting, …\nAn <code>AnsiByteString</code> represents a formatted series of bytes.  …\nA function to construct an <code>AnsiByteStrings</code> instance.\nA set of <code>AnsiByteString</code>s collected together, in order to be\nAn <code>AnsiGenericString</code> includes a generic string type and a …\nA set of <code>AnsiGenericStrings</code>s collected together, in order …\nAn ANSI String is a string coupled with the <code>Style</code> to …\nA function to construct an <code>AnsiStrings</code> instance.\nA set of <code>AnsiString</code>s collected together, in order to be …\nColor #0 (foreground code <code>30</code>, background code <code>40</code>).\nColor #4 (foreground code <code>34</code>, background code <code>44</code>).\nA color is one specific type of ANSI escape code, and can …\nColor #6 (foreground code <code>36</code>, background code <code>46</code>).\nColor #0 (foreground code <code>90</code>, background code <code>100</code>).\nThe default color (foreground code <code>39</code>, background codr <code>49</code>).\nA color number from 0 to 255, for use in 256-color terminal\nColor #2 (foreground code <code>32</code>, background code <code>42</code>).\nColor #4 (foreground code <code>94</code>, background code <code>104</code>).\nColor #6 (foreground code <code>96</code>, background code <code>106</code>).\nColor #7 (foreground code <code>97</code>, background code <code>107</code>).\nColor #2 (foreground code <code>92</code>, background code <code>102</code>).\nColor #5 (foreground code <code>95</code>, background code <code>105</code>).\nColor #5 (foreground code <code>95</code>, background code <code>105</code>).\nColor #1 (foreground code <code>91</code>, background code <code>101</code>).\nColor #3 (foreground code <code>93</code>, background code <code>103</code>).\nColor #5 (foreground code <code>35</code>, background code <code>45</code>).\nColor #5 (foreground code <code>35</code>, background code <code>45</code>).\nColor #1 (foreground code <code>31</code>, background code <code>41</code>).\nA 24-bit Rgb color, as specified by ISO-8613-3.\nA style is a collection of properties that can format a …\nColor #7 (foreground code <code>37</code>, background code <code>47</code>).\nColor #3 (foreground code <code>33</code>, background code <code>43</code>).\nBlue\nThe style’s background color, if it has one.\nReturns a <code>Style</code> with the blink property set.\nReturns a <code>Style</code> with the foreground color set to this …\nReturns a <code>Style</code> with the bold property set.\nReturns a <code>Style</code> with the foreground color set to this …\nReturns a style with <em>no</em> properties set. Formatting text …\nReturns a <code>Style</code> with the dimmed property set.\nReturns a <code>Style</code> with the foreground color set to this …\nReturns a <code>Style</code> with the foreground color property set.\nThe style’s foreground color, if it has one.\nYou can turn a <code>Color</code> into a <code>Style</code> with the foreground …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCreates a new Rgb color with three f32 values\nCreates a new Rgb color with a hex code\nGreen\nCreates a grayscale Rgb color\nCreates a grayscale Rgb color with a f32 value\nReturns a <code>Style</code> with the hidden property set.\nReturns a <code>Style</code> with the foreground color set to this …\nThe infix bytes between this style and <code>next</code> style. These …\nThe infix bytes between this color and <code>next</code> color. These …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nWhether this style is blinking.\nWhether this style is bold.\nWhether this style is dimmed.\nWhether this style is hidden.\nWhether this style is italic.\nReturn true if this <code>Style</code> has no actual styles, and can be …\nWhether this style has reverse colors.\nWhether this style is struckthrough.\nWhether this style is underlined.\nReturns a <code>Style</code> with the italic property set.\nReturns a <code>Style</code> with the foreground color set to this …\nCreates a new Rgb color from a [HSL] color Computes the …\nCreates a new Style with no properties set.\nCreates a new Rgb color\nReturns a <code>Style</code> with the foreground color set to this …\nReturns a <code>Style</code> with the background color property set.\nReturns a <code>Style</code> with the foreground color set to this …\nPaints the given text with this color, returning an ANSI …\nPaints the given text with this color, returning an ANSI …\nThe prefix bytes for this style. These are the bytes that …\nThe prefix bytes for this color as a <code>Style</code>. These are the …\nRed\nReturns a <code>Style</code> with the reverse property set.\nReturns a <code>Style</code> with the foreground color set to this …\nReturns a <code>Style</code> with the strikethrough property set.\nReturns a <code>Style</code> with the foreground color set to this …\nDirectly access the style\nDirectly access the style mutably\nReturn a substring of the given AnsiStrings sequence, …\nThe suffix for this style. These are the bytes that tell …\nThe suffix for this color as a <code>Style</code>. These are the bytes …\nReturns a <code>Style</code> with the underline property set.\nReturns a <code>Style</code> with the foreground color set to this …\nReturn a concatenated copy of <code>strs</code> without the formatting, …\nReturn the unstyled length of AnsiStrings. This is …\nWrite an <code>AnsiByteString</code> to an <code>io::Write</code>.  This writes the …\nWrite <code>AnsiByteStrings</code> to an <code>io::Write</code>.  This writes the …\nWrite an <code>AnsiByteString</code> to an <code>io::Write</code>.  This writes the …\nWrite <code>AnsiByteStrings</code> to an <code>io::Write</code>.  This writes the …\nLike <code>AnsiString</code>, but only displays the difference between …\nLike <code>AnsiString</code>, but only displays the style prefix.\nThe code to send to reset all styles and return to …\nLike <code>AnsiString</code>, but only displays the style suffix.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nLinear color gradient between two color stops\nComputes the Rgb color between <code>start</code> and <code>end</code> for <code>t</code>\nEnd Color of Gradient\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCreates a new Gradient with two Rgb colors, <code>start</code> and <code>end</code>\nReturns the reverse of <code>self</code>\nStart Color of Gradient")