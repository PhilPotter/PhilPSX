"#version 450 core\n"
"\n"
"// Image corresponding to vram texture\n"
"layout (binding = 1, rgba8ui) uniform uimage2D vramImage;\n"
"\n"
"// Uniforms to control draw process\n"
"layout (location = 5) uniform int dither;\n"
"layout (location = 6) uniform int semiTransparencyEnabled;\n"
"layout (location = 7) uniform int semiTransparencyMode;\n"
"layout (location = 8) uniform int setMask;\n"
"layout (location = 9) uniform int checkMask;\n"
"layout (location = 10) uniform int drawTopLeftX;\n"
"layout (location = 11) uniform int drawTopLeftY;\n"
"layout (location = 12) uniform int drawBottomRightX;\n"
"layout (location = 13) uniform int drawBottomRightY;\n"
"\n"
"// Colour input value\n"
"in vec3 interpolated_colour;\n"
"\n"
"// Function declarations\n"
"bool inDrawingArea(ivec2 pixelCoord);\n"
"\n"
"// Dummy output value\n"
"out vec4 colour;\n"
"\n"
"// Draw pixel to vram texture, correctly applying colour\n"
"void main(void) {\n"
"	// Get coordinate from gl_FragCoord\n"
"	ivec2 tempDrawCoord = ivec2(gl_FragCoord.xy);\n"
"\n"
"	// Declare texture pixel variable and make 0 for now\n"
"	uvec4 texPixel = uvec4(0, 0, 0, 0);\n"
"\n"
"	// Deal with colouring and dithering\n"
"	\n"
"	// Merge pixel with blend colour\n"
"	texPixel.r = int(interpolated_colour.r);\n"
"	texPixel.g = int(interpolated_colour.g);\n"
"	texPixel.b = int(interpolated_colour.b);\n"
"		\n"
"	// Check for dither bit\n"
"	if (dither == 1) {\n"
"\n"
"		// Declare dither pixel as signed int vector as otherwise calculations will be off\n"
"		ivec3 ditherPixel = ivec3(int(texPixel.r), int(texPixel.g), int(texPixel.b));\n"
"\n"
"		// Define dither offset array\n"
"		int ditherArray[4][4];\n"
"		ditherArray[0][0] = -4;\n"
"		ditherArray[0][1] = 2;\n"
"		ditherArray[0][2] = -3;\n"
"		ditherArray[0][3] = +3;\n"
"		ditherArray[1][0] = 0;\n"
"		ditherArray[1][1] = -2;\n"
"		ditherArray[1][2] = 1;\n"
"		ditherArray[1][3] = -1;\n"
"		ditherArray[2][0] = -3;\n"
"		ditherArray[2][1] = 3;\n"
"		ditherArray[2][2] = -4;\n"
"		ditherArray[2][3] = 2;\n"
"		ditherArray[3][0] = 1;\n"
"		ditherArray[3][1] = -1;\n"
"		ditherArray[3][2] = 0;\n"
"		ditherArray[3][3] = -2;\n"
"\n"
"		// Calculate dither column and row\n"
"		int ditherColumn = tempDrawCoord.x % 4;\n"
"		int ditherRow = (511 - tempDrawCoord.y) % 4;        \n"
"\n"
"		// Modify pixel\n"
"		ditherPixel.r += ditherArray[ditherColumn][ditherRow];\n"
"		ditherPixel.g += ditherArray[ditherColumn][ditherRow];\n"
"		ditherPixel.b += ditherArray[ditherColumn][ditherRow];\n"
"		\n"
"		if (ditherPixel.r < 0) {\n"
"			ditherPixel.r = 0;\n"
"		}\n"
"		else if (ditherPixel.r > 0xFF) {\n"
"			ditherPixel.r = 0xFF;\n"
"		}\n"
"		\n"
"		if (ditherPixel.g < 0) {\n"
"			ditherPixel.g = 0;\n"
"		}\n"
"		else if (ditherPixel.g > 0xFF) {\n"
"			ditherPixel.g = 0xFF;\n"
"		}\n"
"\n"
"		if (ditherPixel.b < 0) {\n"
"			ditherPixel.b = 0;\n"
"		}\n"
"		else if (ditherPixel.b > 0xFF) {\n"
"			ditherPixel.b = 0xFF;\n"
"		}\n"
"\n"
"		texPixel.r = uint(ditherPixel.r);\n"
"		texPixel.g = uint(ditherPixel.g);\n"
"		texPixel.b = uint(ditherPixel.b);\n"
"	}\n"
"\n"
"	// Restore colours to original 15-bit format\n"
"	texPixel.r = texPixel.r >> 3;\n"
"	if (texPixel.r > 0x1F) {\n"
"		texPixel.r = 0x1F;\n"
"	}\n"
"	texPixel.g = texPixel.g >> 3;\n"
"	if (texPixel.g > 0x1F) {\n"
"		texPixel.g = 0x1F;\n"
"	}\n"
"	texPixel.b = texPixel.b >> 3;\n"
"	if (texPixel.b > 0x1F) {\n"
"		texPixel.b = 0x1F;\n"
"	}\n"
"\n"
"	// Load existing vram pixel\n"
"	uvec4 vramPixel = imageLoad(vramImage, tempDrawCoord);\n"
"\n"
"	// Handle semi-transparency here if enabled\n"
"	if (semiTransparencyEnabled == 1) {\n"
"			\n"
"		int oldRed = int(vramPixel.r);\n"
"		int oldGreen = int(vramPixel.g);\n"
"		int oldBlue = int(vramPixel.b);\n"
"			\n"
"		int newRed = int(texPixel.r);\n"
"		int newGreen = int(texPixel.g);\n"
"		int newBlue = int(texPixel.b);\n"
"\n"
"		// Do calculation\n"
"		switch (semiTransparencyMode) {\n"
"			case 0: // B/2 + F/2\n"
"				newRed = oldRed / 2 + newRed / 2;\n"
"				newGreen = oldGreen / 2 + newGreen / 2;\n"
"				newBlue = oldBlue / 2 + newBlue / 2;\n"
"				break;\n"
"			case 1: // B + F\n"
"				newRed = oldRed + newRed;\n"
"				newGreen = oldGreen + newGreen;\n"
"				newBlue = oldBlue + newBlue;\n"
"				break;\n"
"			case 2: // B - F\n"
"				newRed = oldRed - newRed;\n"
"				newGreen = oldGreen - newGreen;\n"
"				newBlue = oldBlue - newBlue;\n"
"				break;\n"
"			case 3: // B + F/4\n"
"				newRed = oldRed + newRed / 4;\n"
"				newGreen = oldGreen + newGreen / 4;\n"
"				newBlue = oldBlue + newBlue / 4;\n"
"				break;\n"
"		}\n"
"\n"
"		// Saturate pixel\n"
"		if (newRed < 0) {\n"
"			newRed = 0;\n"
"		}\n"
"		else if (newRed > 31) {\n"
"			newRed = 31;\n"
"		}\n"
"\n"
"		if (newGreen < 0) {\n"
"			newGreen = 0;\n"
"		}\n"
"		else if (newGreen > 31) {\n"
"			newGreen = 31;\n"
"		}\n"
"\n"
"		if (newBlue < 0) {\n"
"			newBlue = 0;\n"
"		}\n"
"		else if (newBlue > 31) {\n"
"			newBlue = 31;\n"
"		}\n"
"\n"
"		// Store new pixel values\n"
"		texPixel.r = newRed;\n"
"		texPixel.g = newGreen;\n"
"		texPixel.b = newBlue;\n"
"	}\n"
"\n"
"	// Set mask bit if enabled\n"
"	if (setMask == 1) {\n"
"		texPixel.a = 0x1;\n"
"	}\n"
"	\n"
"	// Check vram pixel if enabled, else just merge, also checking new pixel is in draw area\n"
"	bool inArea = inDrawingArea(tempDrawCoord);\n"
"	if (checkMask == 1) {\n"
"		if (vramPixel.a != 1 && inArea) {\n"
"			imageStore(vramImage, tempDrawCoord, texPixel);\n"
"		}\n"
"	}\n"
"	else if (inArea) {\n"
"		imageStore(vramImage, tempDrawCoord, texPixel);\n"
"	}\n"
"	\n"
"	// Set dummy output value\n"
"	colour = vec4(0.0, 0.0, 0.0, 0.0);\n"
"}\n"
"\n"
"// Tells us if a pixel is in the drawing area\n"
"bool inDrawingArea(ivec2 pixelCoord) {\n"
"	bool retVal = false;\n"
"	if (pixelCoord.x >= drawTopLeftX && pixelCoord.x <= drawBottomRightX &&\n"
"		pixelCoord.y <= drawTopLeftY && pixelCoord.y >= drawBottomRightY) {\n"
"		retVal = true;\n"
"	}\n"
"\n"
"	return retVal;\n"
"}\n"