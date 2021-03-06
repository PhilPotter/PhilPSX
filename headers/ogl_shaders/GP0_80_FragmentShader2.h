/*
 * This header file provides the second OpenGL fragment shader for the GP0_80
 * routine.
 * 
 * GP0_80_FragmentShader2.h - Copyright Phillip Potter, 2020, under GPLv3
 */
#ifndef PHILPSX_GP0_80_FRAGMENTSHADER2
#define PHILPSX_GP0_80_FRAGMENTSHADER2

static const char *GPU_getGP0_80_FragmentShader2Source(void)
{
	return
	"#version 450 core\n"
	"\n"
	"// Images corresponding to temp draw texture and vram texture\n"
	"layout (binding = 0, rgba8ui) uniform uimage2D tempDrawImage;\n"
	"layout (binding = 1, rgba8ui) uniform uimage2D vramImage;\n"
	"\n"
	"// Uniforms to control copy process\n"
	"layout (location = 0) uniform int xOffset;\n"
	"layout (location = 1) uniform int yOffset;\n"
	"layout (location = 2) uniform int setMask;\n"
	"layout (location = 3) uniform int checkMask;\n"
	"\n"
	"// Dummy output value\n"
	"out vec4 colour;\n"
	"\n"
	"// Convert pixel format and store in vram texture\n"
	"void main(void) {\n"
	"	// Get coordinate from gl_FragCoord and apply offset to correctly\n"
	"	// reference temp draw texture\n"
	"	ivec2 tempDrawCoord = ivec2(gl_FragCoord.xy);\n"
	"	tempDrawCoord.x -= xOffset;\n"
	"	tempDrawCoord.y -= yOffset;\n"
	"\n"
	"	// Get coordinate from gl_FragCoord and correctly reference\n"
	"	// vram texture\n"
	"	ivec2 vramCoord = ivec2(gl_FragCoord.xy);\n"
	"\n"
	"	// Load temp draw and vram pixel\n"
	"	uvec4 tempDrawPixel = imageLoad(tempDrawImage, tempDrawCoord);\n"
	"	uvec4 vramPixel = imageLoad(vramImage, vramCoord);\n"
	"\n"
	"	// Set mask bit if enabled\n"
	"	if (setMask == 1) {\n"
	"		tempDrawPixel.a = 0x1;\n"
	"	}\n"
	"\n"
	"	// Check vram pixel if enabled, else just merge\n"
	"	if (checkMask == 1) {\n"
	"		if (vramPixel.a != 1) {\n"
	"			imageStore(vramImage, vramCoord, tempDrawPixel);\n"
	"		}\n"
	"	}\n"
	"	else {\n"
	"		imageStore(vramImage, vramCoord, tempDrawPixel);\n"
	"	}\n"
	"\n"
	"	// Set dummy output value\n"
	"	colour = vec4(0.0, 0.0, 0.0, 0.0);\n"
	"}\n";
}

#endif
