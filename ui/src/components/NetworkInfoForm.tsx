import { JSONSchema7 as JSONSchema } from 'json-schema';
import * as React from 'react';
import { Flex, Form, Heading, RenditionUiSchema } from 'rendition';
import { Network, NetworkInfo } from './App';

const getSchema = (): JSONSchema => ({});

const getUiSchema = (): RenditionUiSchema => ({});

const isEnterpriseNetwork = (
	networks: Network[],
	selectedNetworkSsid?: string,
) => {
	return networks.some(
		(network) =>
			network.ssid === selectedNetworkSsid && network.security === 'enterprise',
	);
};

interface NetworkInfoFormProps {
	onSubmit: (data: NetworkInfo) => void;
}

export const NetworkInfoForm = ({ onSubmit }: NetworkInfoFormProps) => {
	const [data, setData] = React.useState<NetworkInfo>({});

	return (
		<Flex
			flexDirection="column"
			alignItems="center"
			justifyContent="center"
			m={4}
			mt={5}
		>
			<Heading.h3 align="center" mb={4}>
				Click the below button to reset this device's network settings to DHCP.
				Any static IP settings will be lost.
			</Heading.h3>

			<style
				dangerouslySetInnerHTML={{
					__html: `
					.mysubmit {
						background-color: #E63D44;
						border-color: #E63D44;
					}
					.mysubmit:hover, .mysubmit:focus, .mysubmit:active {
						background-color: #EB656B !important;
						border-color: #EB656B !important;
					}
				`,
				}}
			/>
			<Form
				width={['100%', '80%', '60%', '40%']}
				onFormChange={({ formData }) => {
					setData(formData);
				}}
				onFormSubmit={({ formData }) => onSubmit(formData)}
				value={data}
				schema={getSchema()}
				uiSchema={getUiSchema()}
				submitButtonProps={{
					width: '60%',
					mx: '20%',
					mt: 3,
					className: 'mysubmit',
				}}
				submitButtonText={'Reset to DHCP'}
			/>
		</Flex>
	);
};
