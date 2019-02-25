using MediatR;
using QCExtensions.Application.Items.Models;

namespace QCExtensions.Application.Items.Queries.GetImage
{
	public class GetImageQuery : IRequest<byte[]>
	{
		public int ImageId { get; set; }
	}
}
