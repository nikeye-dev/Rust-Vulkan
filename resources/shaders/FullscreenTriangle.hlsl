struct VS_INPUT
{
    uint vertexId : SV_VertexID;
};

struct VS_OUTPUT
{
    float4 position: POSITION;
};

VS_OUTPUT main(const VS_INPUT input)
{
    VS_OUTPUT result;

    float2 xy = float2((input.vertexId << 1) & 2, input.vertexId & 2);
    result.position = float4(xy * float2(2, -2) + float2(-1, 1), 0, 1);

    return result;
}