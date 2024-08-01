#include "common.hlsl"

struct PS_INPUT
{
    float4 position: SV_Position;
    float4 fragColor: COLOR0;
//     float4 worldPos: TEXCOORD1;
};

struct PS_OUTPUT
{
    float4 color: SV_Target;
};

PS_OUTPUT main(PS_INPUT input)
{
    PS_OUTPUT result;
    result.color = input.fragColor;

    return result;
}