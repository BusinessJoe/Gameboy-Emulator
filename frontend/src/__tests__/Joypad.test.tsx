import renderer from 'react-test-renderer';
import Joypad from '../components/Joypad';
import React from 'react';
import { render } from '@testing-library/react';

it('maps keys correctly', () => {
    const screenRef = { current: document.createElement('canvas') };
    const mock = jest.fn();
    const component = renderer.create(
        <Joypad focusRef={screenRef} onJoypadInput={mock} />
    )
    let tree = component.toJSON();
    expect(tree).toMatchSnapshot();
});