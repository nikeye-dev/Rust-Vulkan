#define PI 3.1415926535897932
#define CM_TO_SKY_UNIT 0.00001f

struct Transformation
{
    float4x4 view;
    float4x4 projection;
};

struct ViewState
{
	float4 worldCameraOrigin;
	float4 atmosphereLightDirection;
	float4 atmosphereLightIlluminanceOuterSpace;
};

ConstantBuffer<Transformation> transform : register(b0);
ConstantBuffer<ViewState> resolvedView : register(b1);
