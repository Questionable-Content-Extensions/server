using AutoMapper;
using QCExtensions.Application.Interfaces.Mapping;
using QCExtensions.Domain.Entities;

namespace QCExtensions.Application.Comics.Models
{
	public class ComicListDto
	{
		public int Comic { get; set; }
		public string Title { get; set; }
	}
}
