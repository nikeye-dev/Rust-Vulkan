#include "common.hlsl"

struct FSampleData
{
	float4 planetPos;
	float planetRadius;
	float atmosphereThickness;
	float sampleCount;
	float sampleCountLight;
	float scale;

	float4 lightDir;
	float4 lightIntensity;
};

struct FMedium
{
	float scaleHeightR;
	float scaleHeightM;
	
	//Rayleigh
	float4 scatteringR;
	float4 absorptionR;
	float4 extinctionR;

	//Mie
	float4 scatteringM;
	float4 absorptionM;
	float4 extinctionM;
};

ConstantBuffer<FMedium> medium : register(b2);
ConstantBuffer<FSampleData> sampleData : register(b3);

float2 RayIntersectSphere(float3 rayOrigin, float3 rayDirection, float4 sphere)
{
	float3 localPosition = rayOrigin - sphere.xyz;
	float localPositionSqr = dot(localPosition, localPosition);

	float3 quadraticCoef;
	quadraticCoef.x = dot(rayDirection, rayDirection);
	quadraticCoef.y = 2 * dot(rayDirection, localPosition);
	quadraticCoef.z = localPositionSqr - sphere.w * sphere.w;

	float discriminant = quadraticCoef.y * quadraticCoef.y - 4 * quadraticCoef.x * quadraticCoef.z;

	float2 intersections = -1;

	// Only continue if the ray intersects the sphere
	if (discriminant >= 0)
	{
		float SqrtDiscriminant = sqrt(discriminant);
		intersections = (-quadraticCoef.y + float2(-1, 1) * SqrtDiscriminant) / (2 * quadraticCoef.x);
	}

	return intersections;
}

// Theta - angle between light direction and view direction
float PhaseRayleigh(const float cosTheta)
{
	return (3.0f / (16.0f * PI)) * (1.0f + cosTheta * cosTheta);
}

float3 SingleScattering(const float3 worldPos, float3 viewPos)
{
	const float3 viewDir = normalize(worldPos - viewPos);
	viewPos = (viewPos - sampleData.planetPos.xyz) * sampleData.scale;

	const float4 atmosphere = float4(0, 0, 0, sampleData.planetRadius + sampleData.atmosphereThickness);
	
	float2 p = RayIntersectSphere(viewPos, viewDir, atmosphere);
	if(p.x < 0 && p.y < 0)
		return 0;

	float2 pPlanet = RayIntersectSphere(viewPos, viewDir, float4(0, 0, 0, sampleData.planetRadius));
	if(pPlanet.x > 0)
		p.y = pPlanet.x;
	
	p.x = max(p.x, 0.0f);
	p.y = min(p.y, 9000000.0f);

	//Accumulated light
	float3 a = 0;
	
	float opticalDepth = 0;
	float currentSample = p.x;
	// Sample size
	const float ds = (p.y - p.x) / float(sampleData.sampleCount);

	for(int i = 0; i < sampleData.sampleCount; i++)
	{
		//Sample point X
		const float3 x = viewPos + viewDir * (currentSample + ds * 0.5);

		//Height/Altitude
		const float h = length(x) - sampleData.planetRadius;

		const float densityX = exp(-h / medium.scaleHeightR) * ds;
		opticalDepth += densityX;

		//Light transmittance
		float2 pLight = RayIntersectSphere(x, sampleData.lightDir.xyz, atmosphere);
		
		float currentSampleLight = 0;
		float opticalDepthLight = 0;

		const float dsLight = pLight.y / float(sampleData.sampleCountLight);

		bool underground = false;
		
		for(int j = 0; j < sampleData.sampleCountLight; j++)
		{
			//ToDo: Unify float4. Light dir negated here?
			const float3 xLight = x + sampleData.lightDir.xyz * (currentSampleLight + dsLight * 0.5);
			const float hLight = length(xLight) - sampleData.planetRadius;
			if(hLight < 0)
			{
				underground = true;
				break;
			}
			
			const float densityLight = exp(-hLight / medium.scaleHeightR) * dsLight;

			opticalDepthLight += densityLight;
			currentSampleLight += dsLight;
		}
		//============ Light

		if(!underground)
		{
			const float3 transmittance = exp(-medium.extinctionR.xyz * (opticalDepth + opticalDepthLight));
			a += transmittance * densityX;
		}

		currentSample += ds;
	}

	const float cosTheta = dot(sampleData.lightDir.xyz, viewDir);
	const float phaseR = PhaseRayleigh(cosTheta);
	float3 l = sampleData.lightIntensity.xyz * (phaseR * medium.scatteringR.xyz * a);
	
	return l;
}

struct PS_INPUT
{
    float4 fragColor: COLOR;
    float4 worldPos : TEXCOORD0;
};

struct PS_OUTPUT
{
    float4 Color : SV_Target;
};

PS_OUTPUT main(PS_INPUT input)
{
	PS_OUTPUT result;

	result.Color = float4(SingleScattering(input.worldPos.xyz, resolvedView.worldCameraOrigin.xyz), 1.0);

	return result;
}
