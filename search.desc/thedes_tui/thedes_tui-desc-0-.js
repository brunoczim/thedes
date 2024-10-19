searchState.loadedDescShard("thedes_tui", 0, "Alignment, margin and other settings for texts.\nThis module provides colors that are usable with the …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nMakes a coordinate pair that contains the margin …\nMakes a coordinate pair that contains the margin …\nMakes a coordinate pair that contains the maxima sizes.\nMakes a coordinate pair that contains the minima sizes.\nMakes a coordinate pair that contains the actual sizes.\nCreates a style with the given colors.\nSets alignment. Numerator and align_denomominator are used …\nSets bottom margin.\nUpdates the style to the given color updater.\nSets left margin.\nSets maximum height.\nSets maximum width.\nSets minimum height.\nSets minimum width.\nSets right margin.\nSets top margin.\nAdapts the brightness of the background color to match the …\nAdapts the brightness of the foreground color to match the …\nA trait for types that can approximate their brightness.\n16 Basic colors.\nA basic color. Totals 16 colors. By far, the most portable …\nA basic color used by the terminal.\nBlack.\nThe brightness of a color.\nA color usable in the terminal.\nThe kind of a color. <code>enum</code> representation of an 8-bit color.\nA pair of colors (foreground and background).\nContrasts the brightness of the background color against …\nContrasts the brightness of the foreground color against …\nDark blue/blue.\nDark cyan/cyan.\nDark gray/light black.\nDark green/green.\nDark magenta/magenta.\nDark red/red.\nDark yellow/yellow.\nANSI 8-bit color. Totals 256 colors: 16 basic colors …\nAn 8-bit encoded color for the terminal.\n24 Gray-scale colors.\nA gray-scale color. Goes from white, to gray, to black.\nHalf of maximum gray-scale brightness (gray).\nLight blue.\nLight cyan.\nLight gray/dark white.\nLight green.\nLight magenta.\nLight red.\nLight yellow.\nMaximum brightness (i.e. white).\nMaximum gray-scale brightness (white).\nMinimum brightness (i.e. dark).\nMinimum gray-scale brightness (0, black).\nA function that updates a [<code>Color2</code>].\n216 Legacy RGB colors.\nCommon 24 bit RGB colors (Red-Green-Blue), each channel a …\nUpdates the background of a pair of colors ([<code>Color2</code>]) to …\nUpdates the foreground of a pair of colors ([<code>Color2</code>]) to …\nWhite\nApproximate the brightness of the color.\nThe background of this pair.\nCreates an 8-bit color that is basic.\nReturns the brightness of this color.\nCreates an 8-bit color that is legacy RGB.\nReturns the color code.\nThe foreground of this pair.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCreates an 8-bit color that is gray-scale.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nConverts to en <code>enum</code> representation.\nLevel of brightness. The lower the brightness, the darker …\nReceives a pair of color and yields a new one.\nCreates a new <code>LegacyRgbColor</code> given its components.\nCreates a new gray-scale color given its brightness.\nJust a convenience method for creating color pairs with …\nSet the approximate brightness of the color.\nCreates a new gray-scale color given its brightness. …\nLike <code>Self::set_approx_brightness</code> but takes and returns <code>self</code>\nLike <code>Self::set_approx_brightness</code> but takes and returns <code>self</code>\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nAn INFO dialong: just shows a message.\nThis module exports a simple input dialog and related …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nAn info dialog, with just an Ok option.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nA dialog asking for user input, possibly filtered.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nGeneric initialization. Should not be called directly, but …\nReturned when user cancels this action.\nAn item of a prompt about a dangerous action.\nMenu selection runner.\nReturned when user confirms this action.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nThe backspace key\nA regular, unicode character. E.g. <code>Key::Char(&#39;a&#39;)</code> or …\nThe delete key\nThe down arrow key.\nThe enter key. Preferred over <code>Char(&#39;\\n&#39;)</code>.\nThe escape key.\nA generic event type.\nA supported pressed key.\nUser pressed key.\nAn event fired by a key pressed by the user.\nThe left arrow key.\nUser pasted a string.\nAn event fired by the user pasting data.\nThe right arrow key.\nThe up arrow key.\nWhether alt is modifiying the key (pressed).\nWhether control is modifiying the key (pressed).\nData pasted by the user.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nKey pressed by the user.\nWhether shift is modifiying the key (pressed).\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.")