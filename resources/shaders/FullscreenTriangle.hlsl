#include "common.hlsl"

struct VS_INPUT
{
    float3 position : POSITION;
    float4 color : COLOR0;
//     uint vertexIndex : SV_VertexID;
};

struct VS_PUSH_CONSTANTS
{
    float4x4 model;
};

struct VS_OUTPUT
{
    float4 position: SV_POSITION;
    float4 fragColor: COLOR0;
    float4 worldPos: TEXCOORD0;
};

[[vk::push_constant]] VS_PUSH_CONSTANTS pcs;

VS_OUTPUT main(VS_INPUT input)
{
    VS_OUTPUT result;

//     float2 xy = float2((input.vertexIndex << 1) & 2, input.vertexIndex & 2);
//     float4 position = float4(xy * float2(2, -2) + float2(-1, 1), 0, 1);

    float4 pos = float4(input.position, 1.0);
    pos = mul(pcs.model, pos);

    result.worldPos = pos;

    pos = mul(transform.view, pos);
    pos = mul(transform.projection, pos);
    result.position = pos;

//     float4x4 mvp = mul(mul(transform.projection, transform.view), pcs.model);
//     result.position = mul(mvp, float4(input.position, 1.0));

    result.fragColor = input.color;

    return result;
}